import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { OfficeComponent } from './office.component';

describe('OfficeComponent', () => {
  let component: OfficeComponent;
  let fixture: ComponentFixture<OfficeComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [OfficeComponent, RouterTestingModule],
    }).compileComponents();

    fixture = TestBed.createComponent(OfficeComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
