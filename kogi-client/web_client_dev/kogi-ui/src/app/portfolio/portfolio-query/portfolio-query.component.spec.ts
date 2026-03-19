import { ComponentFixture, TestBed } from '@angular/core/testing';

import { PortfolioQueryComponent } from './portfolio-query.component';

describe('PortfolioQueryComponent', () => {
  let component: PortfolioQueryComponent;
  let fixture: ComponentFixture<PortfolioQueryComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [PortfolioQueryComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(PortfolioQueryComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
