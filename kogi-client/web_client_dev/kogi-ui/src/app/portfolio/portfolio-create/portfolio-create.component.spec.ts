import { ComponentFixture, TestBed } from '@angular/core/testing';
import { RouterTestingModule } from '@angular/router/testing';

import { PortfolioCreateComponent } from './portfolio-create.component';

describe('PortfolioCreateComponent', () => {
  let component: PortfolioCreateComponent;
  let fixture: ComponentFixture<PortfolioCreateComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioCreateComponent, RouterTestingModule],
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioCreateComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
